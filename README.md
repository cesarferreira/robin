# robin
> Run scripts on any project

[![Build Status](https://travis-ci.org/cesarferreira/robin.svg?branch=master)](https://travis-ci.org/cesarferreira/robin)
[![npm](https://img.shields.io/npm/dt/robin-cli-tool.svg)](https://www.npmjs.com/package/robin-cli-tool)
[![npm](https://img.shields.io/npm/v/robin-cli-tool.svg)](https://www.npmjs.com/package/robin-cli-tool) 

<p align="center">
  <img src="media/terminal_ss.png" width="100%" />
</p>

All of the above was generated based on this `.robin.config.json`
file at the root of the project:
```json
{
    "scripts": {
      "clean": "flutter clean && rm-rf ./src/gen/",
      "release": "fastlane ios app_distribution release --{{env}} --rollout=1'",
      "release testflight": "fastlane ios release -e={{env}}'",
    }
}
```  
No need to re-generate / compile any code, it will read your `.robin.config.json` every time you run a command.

## Reason
> Every project has a different way of deploying/releasing/cleaning/etc. By maintaining a simple json file with all the available tasks for this project everyone on the team can run/add/ edit the available tasks for the project on their own machine.

## Install

```sh
npm install -g robin-cli-tool
```

## Usage

```sh
robin init
```

Creates a template `.robin.config.json` in your current folder.
<!-- We can be smart and insert deploy prod if we detect it's flutter, has fastlane? we can pre-populate -->

```json
{
    "scripts": {
      "clean": "...",
      "deploy staging": "echo 'ruby deploy tool --staging'",
      "deploy production": "...",
      "release beta": "...",
      "release alpha": "...",
      "release dev": "..."
    }
  }
  
```

Example: 
```sh
robin release beta # Would run your script to release your app to beta
robin deploy staging # Would deploy your server to staging environment
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

-----------
## Ideas

## Passing params

This config:
```json
{
    "scripts": {
      "clean": "flutter clean && rm-rf ./output/",
      "release": "ruby deploy_tool --{{env}}'",
      "release testflight": "fastlane ios release -e={{env}}'",
    }
}
```  

Would make this possible:
```sh
# clean your builds
robin clean

# deploy the app to the store
robin release --env=staging
robin release --env=production
robin release --env=dev

# release an alpha build
robin release testflight --env=alpha
```



## IDEAS (not implemented yet)

Giving the `.robin.config.json`:

```json
{
    "scripts": {
      "deploy staging": "echo 'ruby deploy tool --staging'",
      "deploy production": "...",
      "clean": "...",
      "release beta": "...",
      "release alpha": "..."
    }
}
  
```

Writing: 
```sh
robin deploy 
```

Will suggest:
- `robin deploy staging`
- `robin deploy production`

Unless there's a `robin deploy` in your scripts list


## Interactive mode

```sh
robin --interactive 
```

So we can fuzzy search the available tasks

<p align="center"><img width="100%"src="https://github.com/cesarferreira/purrge/raw/master/extras/anim.gif"></p>




## Have init templates

```sh
robin init --android
robin init --ios
robin init --flutter
robin init --rails
```


## Created by
[Cesar Ferreira](https://cesarferreira.com)

## License
MIT © [Cesar Ferreira](http://cesarferreira.com)
