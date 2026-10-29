# Import from the NixOS configuration (P-0043).
{ ... }:
{
  services.udev.extraRules = builtins.readFile ./serial.rules;
}
