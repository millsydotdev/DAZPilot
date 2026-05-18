#ifndef SDK_PARSER_H
#define SDK_PARSER_H

#include <string>
#include <vector>
#include <map>
#include <fstream>
#include <regex>
#include <iostream>
#include <sstream>

struct SdkMethod {
    std::string name;
    std::string return_type;
    std::vector<std::string> parameters;
    std::string description;
    std::string access; // public, private, protected, slot
};

struct SdkEnum {
    std::string name;
    std::vector<std::pair<std::string, std::string>> values; // name, value
};

struct SdkClass {
    std::string name;
    std::string file;
    std::string description;
    std::vector<std::string> parents;
    std::vector<SdkMethod> methods;
    std::vector<SdkEnum> enums;
    std::vector<std::string> related_classes;
};

struct SdkIndex {
    std::vector<SdkClass> classes;
    std::map<std::string, std::vector<std::string>> inheritance;
};

class SdkHeaderParser {
public:
    SdkHeaderParser(const std::string& sdk_include_path);
    SdkIndex parseAllHeaders();
    SdkClass parseHeader(const std::string& header_path);
    void saveToJson(const std::string& output_path);

private:
    std::string sdk_path;
    std::vector<std::string> getHeaderFiles();
    std::string extractClassName(const std::string& line);
    std::vector<std::string> extractParentClasses(const std::string& line);
    std::string extractMethodSignature(const std::string& line);
    SdkMethod parseMethod(const std::string& line);
    std::string extractEnumName(const std::string& line);
    SdkEnum parseEnum(const std::vector<std::string>& lines, size_t& pos);
    std::string extractDescription(const std::vector<std::string>& lines, size_t pos);
    std::string cleanType(const std::string& type);
    bool isValidMethod(const std::string& line);
    std::string extractComment(const std::string& line);
};

#endif // SDK_PARSER_H