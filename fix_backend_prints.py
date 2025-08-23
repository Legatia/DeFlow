#!/usr/bin/env python3
import os
import re
import glob

def fix_missing_println(file_path):
    """Fix missing ic_cdk::println! statements in Rust files."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Pattern to find standalone quoted strings (likely missing println!)
        # Look for lines that have quotes but no println! before them
        pattern = r'^(\s+)(".*?"(?:,\s*[^;)]+)*);?\s*$'
        
        lines = content.split('\n')
        fixed_lines = []
        i = 0
        
        while i < len(lines):
            line = lines[i]
            
            # Check if this line looks like a missing println statement
            # - starts with whitespace and a quote
            # - not already part of a println or other macro
            # - not a return statement or other valid usage
            if (re.match(r'^\s+"', line) and 
                'ic_cdk::println!' not in line and
                'return ' not in line and
                'format!' not in line and
                'panic!' not in line and
                'assert!' not in line and
                '= "' not in line and
                'let ' not in line and
                'const ' not in line):
                
                # Look ahead to see if the next few lines continue the pattern
                continuation_lines = []
                j = i + 1
                while j < len(lines) and (lines[j].strip().endswith(',') or 
                                        (lines[j].strip().endswith(');') and not lines[j].strip().startswith(')'))):
                    continuation_lines.append(lines[j])
                    j += 1
                
                # Extract the indentation
                indent = re.match(r'^(\s*)', line).group(1)
                
                # Reconstruct the println statement
                if continuation_lines:
                    # Multi-line format string
                    println_line = f"{indent}ic_cdk::println!({line.strip()}"
                    fixed_lines.append(println_line)
                    for cont_line in continuation_lines[:-1]:
                        fixed_lines.append(cont_line)
                    # Fix the last line to close the println properly
                    last_line = continuation_lines[-1]
                    if last_line.strip().endswith(');'):
                        fixed_lines.append(last_line)
                    else:
                        fixed_lines.append(last_line.rstrip().rstrip(',') + ');')
                    i = j
                else:
                    # Single line
                    quote_content = line.strip()
                    if quote_content.endswith(';'):
                        quote_content = quote_content[:-1]
                    fixed_line = f"{indent}ic_cdk::println!({quote_content});"
                    fixed_lines.append(fixed_line)
                    i += 1
            else:
                fixed_lines.append(line)
                i += 1
        
        new_content = '\n'.join(fixed_lines)
        
        # Only write if content changed
        if new_content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"Fixed: {file_path}")
            return True
        
        return False
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    backend_dir = "/Users/zhang/Desktop/ICP/DeFlow/src/DeFlow_backend/src"
    
    # Find all Rust files
    rust_files = []
    for root, dirs, files in os.walk(backend_dir):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    print(f"Found {len(rust_files)} Rust files to process...")
    
    fixed_count = 0
    for file_path in rust_files:
        if fix_missing_println(file_path):
            fixed_count += 1
    
    print(f"Fixed {fixed_count} files")

if __name__ == "__main__":
    main()