# Permission System Documentation

## Overview

The DazPilot uses a comprehensive, database-driven permission system that controls access to all features and operations. All permissions are configurable with no hardcoded values.

---

## Permission Architecture

### Core Principles

1. **Dynamic Discovery** - All permissions stored in database, not hardcoded
2. **Role-Based Access** - Users assigned to roles with specific permissions
3. **Runtime Prompts** - Sensitive operations require user confirmation
4. **Audit Logging** - All permission actions logged for review

### Permission Categories

| Category | Description |
|----------|-------------|
| SYSTEM | Core application functionality |
| FEATURE | Feature access and usage |
| ASSET | Asset library access |
| OPERATION | Import/export operations |
| AI | AI behavior and learning |
| DAZ3D | Daz3D plugin interaction |
| NETWORK | Network/cloud features |
| RUNTIME | Session-based permissions |

---

## Database Schema

### Permission Definitions Table

```sql
CREATE TABLE permissions (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    default_state TEXT DEFAULT 'prompt',
    requires_prompt BOOLEAN DEFAULT false,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### User Roles Table

```sql
CREATE TABLE user_roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_admin BOOLEAN DEFAULT false,
    inherit_from TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Role Permissions Table

```sql
CREATE TABLE role_permissions (
    role_id TEXT NOT NULL,
    permission_id TEXT NOT NULL,
    state TEXT NOT NULL,  -- granted, denied, prompt
    conditions TEXT,
    PRIMARY KEY (role_id, permission_id)
);
```

### User Role Assignments Table

```sql
CREATE TABLE user_role_assignments (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role_id)
);
```

---

## Default Permission Groups

### 1. Basic User Role

| Permission | Default State | Notes |
|------------|---------------|-------|
| feature.scene.create | granted | Can create scenes |
| feature.scene.save | granted | Can save scenes |
| feature.library.scan | granted | Can scan library |
| feature.library.browse | granted | Can browse assets |
| feature.animation.view | granted | Can view animations |
| feature.animation.create | granted | Can create animations |
| feature.render.preview | granted | Can preview render |
| feature.render.full | granted | Can full render |
| ai.auto_apply | prompt | Needs confirmation |
| ai.learn_patterns | granted | Can learn patterns |
| ai.view_analytics | denied | Not for basic users |
| daz3d.load_assets | granted | Can load assets |
| daz3d.modify_scene | granted | Can modify scenes |
| daz3d.execute_scripts | denied | Security restriction |
| network.cloud_sync | denied | Not enabled |
| network.download | denied | Not enabled |

### 2. Admin Role

| Permission | Default State | Notes |
|------------|---------------|-------|
| system.settings | granted | Full settings access |
| system.manage_users | granted | User management |
| system.view_audit | granted | View audit logs |
| feature.* | granted | All features |
| ai.* | granted | Full AI access |
| daz3d.* | granted | Full Daz3D access |
| network.* | granted | Full network access |

---

## Permission Check Flow

```typescript
async function checkPermission(permissionId: string, userId: string): Promise<boolean> {
    // 1. Get user's roles
    const roles = await getUserRoles(userId);

    // 2. Check each role's permission
    for (const role of roles) {
        const state = await getRolePermissionState(role.id, permissionId);

        if (state === 'denied') return false;
        if (state === 'granted') return true;
        if (state === 'prompt') return await promptUser(permissionId);
    }

    // 3. Default to prompt if no roles have permission
    return await promptUser(permissionId);
}
```

---

## Runtime Permission Prompts

When a permission requires user confirmation:

```json
{
  "type": "permission_prompt",
  "title": "Permission Required",
  "message": "The AI wants to automatically apply your preferred shell. Allow?",
  "permission_id": "ai.auto_apply_shell",
  "options": {
    "allow_once": true,
    "allow_always": true,
    "deny": true,
    "deny_always": true
  }
}
```

---

## Permission Audit Logging

All permission actions are logged:

```sql
CREATE TABLE permission_audit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    permission_id TEXT NOT NULL,
    action TEXT NOT NULL,  -- check, grant, deny, request, update
    result TEXT NOT NULL,   -- allowed, denied, error
    ip_address TEXT,
    context TEXT,          -- JSON with additional context
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## Managing Permissions (Admin Functions)

### Grant Permission to Role
```sql
INSERT INTO role_permissions (role_id, permission_id, state)
VALUES ('admin', 'system.settings', 'granted');
```

### Deny Permission to Role
```sql
INSERT INTO role_permissions (role_id, permission_id, state)
VALUES ('basic', 'daz3d.execute_scripts', 'denied');
```

### Update Default Permission
```sql
UPDATE permissions
SET default_state = 'granted'
WHERE id = 'feature.library.scan';
```

---

## User-Facing Permission UI

### Settings Page
- View current permissions
- Request elevated access
- See permission history

### Permission Prompts
- Clear explanation of what's being requested
- One-time vs always options
- Easy to understand language

---

## Best Practices

1. **Least Privilege** - Default to prompt, grant as needed
2. **Audit Regularly** - Review permission usage
3. **Clear Prompts** - Explain why permission is needed
4. **Document Exceptions** - Note any permission workarounds

---

## Dynamic Permission Discovery

All permissions are loaded from database at startup:

```rust
fn load_permissions() -> Vec<Permission> {
    let conn = Database::connect();
    let mut stmt = conn.prepare("SELECT * FROM permissions").unwrap();

    stmt.query_map([], |row| {
        Ok(Permission {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            category: row.get(3)?,
            default_state: row.get(4)?,
            requires_prompt: row.get(5)?,
        })
    }).collect()
}
```

---

## No Hardcoding Verification

To verify no permissions are hardcoded in code:

1. Search codebase for permission strings in code
2. Ensure all permission checks use database lookup
3. Verify permission constants come from config

---

Last Updated: May 2026
