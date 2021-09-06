#!/usr/bin/env node
'use strict';

const meow = require('meow');
const router = require('./src/router');
const updateNotifier = require('update-notifier');
const pkg = require('./package.json');

updateNotifier({ pkg }).notify();

const cli = meow(`
Usage

   $ robin <command> <params>

   $ robin init            # creates a new '.robin.json' config
   $ robin list            # Lists the available commanbds
   
   Examples
   
   $ robin clean           # Runs the clean script (eg: rm -rf node_modules)
   $ robin release ios     # Runs the 'release ios' script (eg: fastlane run testflight)
  
`,
  {
    alias: {
      v: 'version'
    },
    boolean: ['version']
  }
);

if (cli.input.length > 0) {
	router.init(cli.input, cli.flags);
} else {
	cli.showHelp(2);
}