#include "parser.h"
#include <filesystem>
#include <algorithm>
#include <set>

namespace fs = std::filesystem;

SdkHeaderParser::SdkHeaderParser(const std::string& sdk_include_path) 
    : sdk_path(sdk_include_path) {
}

std::vector<std::string> SdkHeaderParser::getHeaderFiles() {
    std::vector<std::string> headers;
    std::set<std::string> seen;
    
    try {
        for (const auto& entry : fs::recursive_directory_iterator(sdk_path)) {
            if (entry.is_regular_file()) {
                std::string path = entry.path().string();
                std::string filename = entry.path().filename().string();
                
                // Only process dz*.h files
                if (filename.substr(0, 2) == "dz" && 
                    filename.find(".h") != std::string::npos &&
                    seen.find(filename) == seen.end()) {
                    seen.insert(filename);
                    headers.push_back(path);
                }
            }
        }
    } catch (const std::exception& e) {
        std::cerr << "Error reading directory: " << e.what() << std::endl;
    }
    
    std::sort(headers.begin(), headers.end());
    return headers;
}

std::string SdkHeaderParser::extractClassName(const std::string& line) {
    std::regex class_regex(R"(class\s+(\w+)\s*:.*?(public|protected|private)?\s*Dz\w+)", std::regex::icase);
    std::smatch match;
    
    if (std::regex_search(line, match, class_regex)) {
        return match[1].str();
    }
    
    // Simpler pattern for Dz classes
    std::regex simple_regex(R"(class\s+(Dz\w+))");
    if (std::regex_search(line, match, simple_regex)) {
        return match[1].str();
    }
    
    return "";
}

std::vector<std::string> SdkHeaderParser::extractParentClasses(const std::string& line) {
    std::vector<std::string> parents;
    std::regex inherit_regex(R"(public\s+(Dz\w+))");
    std::sregex_iterator it(line.begin(), line.end(), inherit_regex);
    std::sregex_iterator end;
    
    while (it != end) {
        parents.push_back((*it)[1].str());
        ++it;
    }
    
    return parents;
}

std::string SdkHeaderParser::extractMethodSignature(const std::string& line) {
    // Remove Q_OBJECT, Q_INTERFACES, etc.
    std::string cleaned = line;
    
    // Remove leading whitespace
    size_t start = cleaned.find_first_not_of(" \t");
    if (start != std::string::npos) {
        cleaned = cleaned.substr(start);
    }
    
    return cleaned;
}

SdkMethod SdkHeaderParser::parseMethod(const std::string& line) {
    SdkMethod method;
    method.access = "public";
    
    // Determine access modifier
    if (line.find("public slots:") != std::string::npos || line.find("public Q_SLOTS") != std::string::npos) {
        method.access = "slot";
    } else if (line.find("public:") != std::string::npos) {
        method.access = "public";
    } else if (line.find("private:") != std::string::npos) {
        method.access = "private";
    } else if (line.find("protected:") != std::string::npos) {
        method.access = "protected";
    }
    
    // Extract return type and method name
    // Pattern: return_type method_name(params);
    std::regex method_regex(R"(([\w:\*&<>\s]+?)\s+(\w+)\s*\(([^)]*)\))");
    std::smatch match;
    
    if (std::regex_search(line, match, method_regex)) {
        method.return_type = cleanType(match[1].str());
        method.name = match[2].str();
        
        std::string params = match[3].str();
        if (!params.empty() && params != "void") {
            std::stringstream ss(params);
            std::string param;
            while (std::getline(ss, param, ',')) {
                param.erase(0, param.find_first_not_of(" \t"));
                param.erase(param.find_last_not_of(" \t") + 1);
                if (!param.empty()) {
                    method.parameters.push_back(param);
                }
            }
        }
    }
    
    // Extract comment/description
    method.description = extractComment(line);
    
    return method;
}

std::string SdkHeaderParser::extractEnumName(const std::string& line) {
    std::regex enum_regex(R"(enum\s+(\w+))");
    std::smatch match;
    
    if (std::regex_search(line, match, enum_regex)) {
        return match[1].str();
    }
    
    // Check for Q_ENUM
    std::regex qenum_regex(R"(Q_ENUM\s*\((\w+)\))");
    if (std::regex_search(line, match, qenum_regex)) {
        return match[1].str();
    }
    
    return "";
}

SdkEnum SdkHeaderParser::parseEnum(const std::vector<std::string>& lines, size_t& pos) {
    SdkEnum enm;
    bool in_enum = false;
    
    for (size_t i = pos; i < lines.size() && i < pos + 50; ++i) {
        std::string line = lines[i];
        
        if (line.find("enum") != std::string::npos) {
            std::string name = extractEnumName(line);
            if (!name.empty()) {
                enm.name = name;
            }
            in_enum = true;
            continue;
        }
        
        if (in_enum) {
            if (line.find("};") != std::string::npos) {
                break;
            }
            
            // Extract enum values like: ValueName = 0,
            std::regex value_regex(R"((\w+)\s*(?:=\s*(\w+))?)");
            std::smatch match;
            if (std::regex_search(line, match, value_regex)) {
                std::string val_name = match[1].str();
                std::string val_value = match.size() > 2 ? match[2].str() : "";
                
                // Skip keywords
                if (val_name != "enum" && val_name != "Q_ENUM" && val_name != "Q_ENUMS") {
                    enm.values.push_back({val_name, val_value});
                }
            }
        }
    }
    
    return enm;
}

std::string SdkHeaderParser::extractDescription(const std::vector<std::string>& lines, size_t pos) {
    // Look for Doxygen comments before the declaration
    if (pos > 0) {
        for (size_t i = pos - 1; i > 0 && pos - i < 5; --i) {
            std::string line = lines[i];
            if (line.find("@brief") != std::string::npos || line.find("\\brief") != std::string::npos) {
                size_t start = line.find("@brief") != std::string::npos ? line.find("@brief") + 6 : line.find("\\brief") + 6;
                std::string desc = line.substr(start);
                desc.erase(0, desc.find_first_not_of(" \t"));
                return desc;
            }
            if (line.find("///") != std::string::npos || line.find("/**") != std::string::npos) {
                std::string comment = line;
                comment = std::regex_replace(comment, std::regex(R"(///\s*)"), "");
                comment = std::regex_replace(comment, std::regex(R"(\*/)"), "");
                comment = std::regex_replace(comment, std::regex(R"(//)"), "");
                return comment;
            }
        }
    }
    return "";
}

std::string SdkHeaderParser::cleanType(const std::string& type) {
    std::string cleaned = type;
    cleaned = std::regex_replace(cleaned, std::regex(R"(\s+)"), " ");
    cleaned.erase(0, cleaned.find_first_not_of(" \t"));
    cleaned.erase(cleaned.find_last_not_of(" \t") + 1);
    return cleaned;
}

bool SdkHeaderParser::isValidMethod(const std::string& line) {
    // Skip comments and preprocessor
    if (line.substr(0, 2) == "//" || line.substr(0, 1) == "#") {
        return false;
    }
    
    // Must have a method signature pattern
    if (line.find("(") == std::string::npos) {
        return false;
    }
    
    // Skip constructors/destructors that don't return type
    std::regex constructor_regex(R"(\s+(Dz\w+)\s*\()");
    if (std::regex_match(line, constructor_regex)) {
        return true; // Constructor
    }
    
    // Must have a return type (not just method name)
    std::regex valid_method_regex(R"([\w:\*&<>\s]+\s+\w+\s*\()");
    return std::regex_search(line, valid_method_regex);
}

std::string SdkHeaderParser::extractComment(const std::string& line) {
    size_t comment_pos = line.find("//");
    if (comment_pos != std::string::npos) {
        std::string comment = line.substr(comment_pos + 2);
        comment.erase(0, comment.find_first_not_of(" \t"));
        return comment;
    }
    return "";
}

SdkClass SdkHeaderParser::parseHeader(const std::string& header_path) {
    SdkClass cls;
    cls.file = fs::path(header_path).filename().string();
    
    std::ifstream file(header_path);
    if (!file.is_open()) {
        std::cerr << "Failed to open: " << header_path << std::endl;
        return cls;
    }
    
    std::vector<std::string> lines;
    std::string line;
    while (std::getline(file, line)) {
        lines.push_back(line);
    }
    file.close();
    
    bool in_class = false;
    bool in_public = false;
    std::vector<SdkEnum> enums;
    std::vector<SdkMethod> methods;
    
    for (size_t i = 0; i < lines.size(); ++i) {
        line = lines[i];
        
        // Check for class definition
        if (line.find("class") != std::string::npos && line.find("Dz") != std::string::npos) {
            std::string name = extractClassName(line);
            if (!name.empty() && name.substr(0, 2) == "Dz") {
                cls.name = name;
                cls.parents = extractParentClasses(line);
                cls.description = extractDescription(lines, i);
                in_class = true;
                continue;
            }
        }
        
        if (!in_class || cls.name.empty()) {
            continue;
        }
        
        // Check for access specifiers
        if (line.find("public:") != std::string::npos) {
            in_public = true;
            continue;
        } else if (line.find("private:") != std::string::npos || line.find("protected:") != std::string::npos) {
            in_public = false;
            continue;
        }
        
        // Check for Q_ENUM or Q_ENUMS
        if (line.find("Q_ENUM") != std::string::npos || line.find("Q_ENUMS") != std::string::npos) {
            SdkEnum enm = parseEnum(lines, i);
            if (!enm.name.empty()) {
                enums.push_back(enm);
            }
            continue;
        }
        
        // Check for enum definition
        if (line.find("enum") != std::string::npos) {
            SdkEnum enm = parseEnum(lines, i);
            if (!enm.name.empty()) {
                enums.push_back(enm);
            }
            continue;
        }
        
        // Parse methods
        if (in_public && isValidMethod(line)) {
            SdkMethod method = parseMethod(line);
            if (!method.name.empty() && method.name != "Q_OBJECT") {
                methods.push_back(method);
            }
        }
    }
    
    cls.methods = methods;
    cls.enums = enums;
    
    // Determine related classes from parents and method return types
    std::set<std::string> related;
    for (const auto& parent : cls.parents) {
        if (parent.substr(0, 2) == "Dz") {
            related.insert(parent);
        }
    }
    for (const auto& method : methods) {
        if (method.return_type.substr(0, 2) == "Dz") {
            related.insert(method.return_type);
        }
    }
    cls.related_classes.assign(related.begin(), related.end());
    
    return cls;
}

SdkIndex SdkHeaderParser::parseAllHeaders() {
    SdkIndex index;
    auto headers = getHeaderFiles();
    
    std::cout << "Found " << headers.size() << " header files to parse..." << std::endl;
    
    for (size_t i = 0; i < headers.size(); ++i) {
        if (i % 50 == 0) {
            std::cout << "Parsing header " << i << " of " << headers.size() << ": " 
                      << fs::path(headers[i]).filename().string() << std::endl;
        }
        
        SdkClass cls = parseHeader(headers[i]);
        if (!cls.name.empty()) {
            index.classes.push_back(cls);
            
            // Build inheritance map
            for (const auto& parent : cls.parents) {
                index.inheritance[parent].push_back(cls.name);
            }
        }
    }
    
    std::cout << "Parsed " << index.classes.size() << " classes" << std::endl;
    return index;
}

void SdkHeaderParser::saveToJson(const std::string& output_path) {
    SdkIndex index = parseAllHeaders();
    
    std::ofstream out(output_path);
    if (!out.is_open()) {
        std::cerr << "Failed to open output file: " << output_path << std::endl;
        return;
    }
    
    out << "{\n";
    out << "  \"classes\": [\n";
    
    for (size_t i = 0; i < index.classes.size(); ++i) {
        const auto& cls = index.classes[i];
        out << "    {\n";
        out << "      \"name\": \"" << cls.name << "\",\n";
        out << "      \"file\": \"" << cls.file << "\",\n";
        out << "      \"description\": \"" << cls.description << "\",\n";
        
        out << "      \"parents\": [";
        for (size_t j = 0; j < cls.parents.size(); ++j) {
            out << "\"" << cls.parents[j] << "\"";
            if (j < cls.parents.size() - 1) out << ", ";
        }
        out << "],\n";
        
        out << "      \"methods\": [\n";
        for (size_t j = 0; j < cls.methods.size(); ++j) {
            const auto& m = cls.methods[j];
            out << "        {\"name\": \"" << m.name << "\", \"return_type\": \"" << m.return_type << "\", ";
            out << "\"parameters\": [";
            for (size_t k = 0; k < m.parameters.size(); ++k) {
                out << "\"" << m.parameters[k] << "\"";
                if (k < m.parameters.size() - 1) out << ", ";
            }
            out << "], \"description\": \"" << m.description << "\", \"access\": \"" << m.access << "\"}";
            if (j < cls.methods.size() - 1) out << ",";
            out << "\n";
        }
        out << "      ],\n";
        
        out << "      \"enums\": [\n";
        for (size_t j = 0; j < cls.enums.size(); ++j) {
            const auto& e = cls.enums[j];
            out << "        {\"name\": \"" << e.name << "\", \"values\": [";
            for (size_t k = 0; k < e.values.size(); ++k) {
                out << "{\"" << e.values[k].first << "\", \"" << e.values[k].second << "\"}";
                if (k < e.values.size() - 1) out << ", ";
            }
            out << "]}";
            if (j < cls.enums.size() - 1) out << ",";
            out << "\n";
        }
        out << "      ],\n";
        
        out << "      \"related_classes\": [";
        for (size_t j = 0; j < cls.related_classes.size(); ++j) {
            out << "\"" << cls.related_classes[j] << "\"";
            if (j < cls.related_classes.size() - 1) out << ", ";
        }
        out << "]\n";
        
        out << "    }";
        if (i < index.classes.size() - 1) out << ",";
        out << "\n";
    }
    
    out << "  ]\n";
    out << "}\n";
    
    out.close();
    std::cout << "Saved SDK index to: " << output_path << std::endl;
}

int main(int argc, char* argv[]) {
    if (argc < 3) {
        std::cerr << "Usage: " << argv[0] << " <sdk_include_path> <output_json_path>" << std::endl;
        return 1;
    }
    
    std::string sdk_path = argv[1];
    std::string output_path = argv[2];
    
    SdkHeaderParser parser(sdk_path);
    parser.saveToJson(output_path);
    
    return 0;
}