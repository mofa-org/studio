# Skill maintenance

## 1. Update flow
1. Edit SKILL.md and references.
2. Validate:
  ```bash
  python3 /Users/yao/.codex/skills/.system/skill-creator/scripts/quick_validate.py skills/<skill-folder>
  ```
3. Package:
  ```bash
  python3 /Users/yao/.codex/skills/.system/skill-creator/scripts/package_skill.py skills/<skill-folder>
  ```

## 2. Naming rules
- Skill folder name must match the `name` in frontmatter.
- Use lowercase letters, digits, and hyphens only.
