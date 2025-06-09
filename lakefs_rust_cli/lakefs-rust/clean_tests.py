import sys
import re

with open(sys.argv[1], 'r') as f:
    content = f.read()

# Remove all test modules
pattern = r'#\[cfg\(test\)\]\s*\nmod tests\s*\{[^}]*\}'
while True:
    match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
    if not match:
        break
    
    # Find the actual end of this test module by counting braces
    start = match.start()
    brace_count = 0
    end = start
    
    for i in range(start, len(content)):
        if content[i] == '{':
            brace_count += 1
        elif content[i] == '}':
            brace_count -= 1
            if brace_count == 0:
                end = i + 1
                break
    
    # Remove this test module
    content = content[:start] + content[end:]

# Clean up extra whitespace
content = re.sub(r'\n{3,}', '\n\n', content)
content = content.rstrip() + '\n'

with open(sys.argv[1], 'w') as f:
    f.write(content)
