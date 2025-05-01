#!/usr/bin/env python3
import json
import sys
import os

def migrate_template(file_path):
    # Read the file
    with open(file_path, 'r') as f:
        try:
            data = json.load(f)
        except json.JSONDecodeError:
            print(f"Error: Could not parse JSON in {file_path}")
            return False
    
    # Check if it already has tasks
    if "tasks" in data:
        print(f"File already has tasks: {file_path}")
        return True
    
    # Check if it has scripts
    if "scripts" not in data:
        print(f"No scripts found in {file_path}")
        return False
    
    # Migrate scripts to tasks
    tasks = []
    for name, command in data["scripts"].items():
        # Create a meaningful description based on the name
        words = name.replace(':', ' ').split()
        if len(words) == 1:
            description = f"Run {name}"
        else:
            description = " ".join(word.capitalize() for word in words)
        
        task = {
            "name": name,
            "command": command,
            "description": description
        }
        tasks.append(task)
    
    # Create new data
    new_data = {
        "tasks": tasks
    }
    
    # Keep any other keys that might exist
    for key in data:
        if key != "scripts":
            new_data[key] = data[key]
    
    # Write back
    with open(file_path, 'w') as f:
        json.dump(new_data, f, indent=2)
    
    print(f"Migrated {file_path}")
    return True

if __name__ == "__main__":
    template_dir = "templates"
    templates = [
        "android.json", "go.json", "ios.json", "nextjs.json", 
        "node.json", "python.json", "rails.json", "sample.json"
    ]
    
    for template in templates:
        path = os.path.join(template_dir, template)
        migrate_template(path) 