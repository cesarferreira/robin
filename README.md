# robin
> Run scripts for any project

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
[ 
  "deploy prod": "/bin/ruby deploy tool --production",
  "clean database": "/bin/mysql database --production --delete",
]
```


```sh
robin list # Lists all the available commands
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
