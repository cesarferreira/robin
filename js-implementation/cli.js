#!/usr/bin/env node
"use strict";

const meow = require("meow");
const router = require("./src/router");
const updateNotifier = require("update-notifier");
const pkg = require("./package.json");
const log = console.log;

updateNotifier({ pkg }).notify();

const cli = meow(
  `
Usage

   $ robin <command> <params>

   $ robin init              # creates a new '.robin.json' config
   $ robin --list            # Lists the available commanbds
   $ robin --interactive     # Starts a fuzzy search for available commands
   
   Examples
   
   $ robin clean           # Runs the clean script (eg: rm -rf node_modules)
   $ robin release ios     # Runs the 'release ios' script (eg: fastlane run testflight)
  
`,
  {
    alias: {
      v: "version",
      i: "interactive",
      l: "list",
    },
    boolean: ["version"],
    boolean: ["interactive"],
    boolean: ["list"],
  }
);

if (cli.input.length > 0 || Object.keys(cli.flags).length > 0) {
  router.init(cli.input, cli.flags);
} else {
  cli.showHelp(2);
}
