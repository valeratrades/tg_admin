# tg_admin configuration in Nix format
# This file is evaluated with `nix eval --json --impure` so you can use:
# - Environment variables: builtins.getEnv "VAR_NAME"
# - Nix language features like conditionals, imports, etc.
{
  tg_token = builtins.getEnv "TG_TOKEN_TEST";
  admin_list = [ 123456789 987654321 ];
}
