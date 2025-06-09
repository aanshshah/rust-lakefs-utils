import re

# Read the file
with open('crates/lakefs-api/src/client.rs', 'r') as f:
    content = f.read()

# Find the first test module
first_test_start = content.find('#[cfg(test)]')
if first_test_start == -1:
    print("No test module found")
    exit(1)

# Find where the first test module ends
brace_count = 0
in_module = False
first_test_end = first_test_start

for i in range(first_test_start, len(content)):
    if content[i] == '{':
        brace_count += 1
        in_module = True
    elif content[i] == '}':
        brace_count -= 1
        if in_module and brace_count == 0:
            first_test_end = i + 1
            break

# Find the second test module
second_test_start = content.find('#[cfg(test)]', first_test_end)
if second_test_start == -1:
    print("No duplicate test module found")
    exit(0)

# Extract content up to the second test module
new_content = content[:second_test_start].rstrip()

# Make sure we don't have trailing braces
# Count braces in the remaining content
open_braces = new_content.count('{')
close_braces = new_content.count('}')

# If we have more closing braces, remove the extras from the end
while close_braces > open_braces:
    last_brace = new_content.rfind('}')
    new_content = new_content[:last_brace] + new_content[last_brace+1:]
    close_braces -= 1

# Write the fixed content
with open('crates/lakefs-api/src/client.rs', 'w') as f:
    f.write(new_content + '\n')

print("Fixed crates/lakefs-api/src/client.rs")
