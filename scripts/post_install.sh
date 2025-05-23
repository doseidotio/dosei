#!/bin/bash
set -e

dosei_bin="/Applications/Dosei.app/Contents/Resources/dosei"
dosei_install="${DOSEI_CLI_INSTALL:-$HOME/.dosei}"
bin_dir="$dosei_install/bin"
exe="$bin_dir/dosei"
symlink_dir="/usr/local/bin"
symlink_path="$symlink_dir/dosei"

# Function to display a native macOS dialog for sudo permission
prompt_for_password() {
    osascript -e "do shell script \"$1\" with administrator privileges"
}

# Create the bin directory if it doesn't exist
if [ ! -d "$bin_dir" ]; then
  mkdir -p "$bin_dir"
fi

# Copy the binary to the installation directory
cp "$dosei_bin" "$exe"
chmod +x "$exe"

echo "Dosei CLI was installed successfully to $exe"

# Create a symlink in /usr/local/bin (requires sudo)
if [ -w "$symlink_dir" ]; then
  # If we have write permission to /usr/local/bin
  ln -sf "$exe" "$symlink_path"
  echo "Created symlink at $symlink_path"
else
  # Use osascript to prompt for admin password with a native dialog
  echo "Creating symlink in $symlink_dir (requires administrator privileges)..."

  # The command we want to run with sudo
  sudo_cmd="ln -sf \"$exe\" \"$symlink_path\""

  # Try to use osascript to show a graphical prompt
  if prompt_for_password "$sudo_cmd" >/dev/null 2>&1; then
    echo "Created symlink at $symlink_path"
  else
    echo "Failed to create symlink. You may need to run the following command manually:"
    echo "  sudo ln -sf \"$exe\" \"$symlink_path\""
  fi
fi

# Check if the symlink worked and dosei is now in the PATH
if command -v dosei >/dev/null; then
  echo "Run 'dosei --help' to get started"
else
  echo "NOTE: You may need to open a new terminal window for the 'dosei' command to be available"
  echo "Run '$exe --help' to get started"
fi

echo
echo "Stuck? Join our Discord https://discord.com/invite/BP5aUkhcAh"
