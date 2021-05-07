# robin
> Run scripts for any project


## Reason
> every project has a different way of deploying/releasing/etc
By maintaining a simple json file with all the available tasks for this project everyone on the team can add / edit the available tasks for the project.

### Sharing is caring?
> add it to `.gitignore` or share it with the team
<!-- <p align="center">
  <img src="https://raw.githubusercontent.com/cesarferreira/assets/master/images/screenshot_terminal_hello_world.png" width="100%" />
</p>

[![Build Status](https://travis-ci.org/cesarferreira/robin.svg?branch=master)](https://travis-ci.org/cesarferreira/robin)
[![npm](https://img.shields.io/npm/dt/robin.svg)](https://www.npmjs.com/package/robin)
[![npm](https://img.shields.io/npm/v/robin.svg)](https://www.npmjs.com/package/robin) -->

## Install

```sh
npm install -g robin
```

 ## Usage


```sh
robin init # Creates an empty .robin.config
```
Generates this file:
<!-- We can be smart and insert deploy prod if we detect it's flutter, has fastlane? we can pre-populate -->
`.robin.json`

```json
{
  "scripts": [ 
    "deploy staging": "echo 'ruby deploy tool --staging'",
    "deploy production": "...",
    "clean": "...",
    "release beta": "...",
    "release alpha": "...",
  ]
}
```

Example: 
```sh
robin release beta # Would run your script to release your app to beta
```
--------------

```sh
robin list # Lists all the available commands
```
--------------

```sh
robin add # Adds a command
```

Example: 
```sh
robin add "deploy" "fastlane deliver --submit-to-review" # Adds a deploy command to your current list of commands
```


<!-- 
```

Usage

   $ robin <command> <params>

   $ robin sample <param>             # Uses the <PARAM>
   
 Examples

   $ robin sample TEST                # Uses the TEST
   $ robin sample YOLO                # Uses the YOLO
```  -->

<!-- ## Created by
[Cesar Ferreira](https://cesarferreira.com)

## License
MIT © [Cesar Ferreira](http://cesarferreira.com) -->
