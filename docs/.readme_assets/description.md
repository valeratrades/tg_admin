Manage configuration files via a Telegram bot. Supports TOML, JSON, YAML, and Nix formats.

## Configuration
Create a config file at `~/.config/tg_admin/config.toml`:
```toml
tg_token = "YOUR_BOT_TOKEN"
# Optional: restrict access to specific users (usernames or numeric IDs)
admin_list = ["@your_username", 123456789]
```
