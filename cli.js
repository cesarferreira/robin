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

   $ robin sample <param>             # Uses the <PARAM>
   $ robin other <param>              # Other the <PARAM>
   $ robin another <param>            # Another the <PARAM>
   
 Examples

   $ robin sample TEST                # Uses the TEST
   $ robin sample YOLO                # Uses the YOLO
   $ robin other YOLO                 # Uses the YOLO
   $ robin another YOLO               # Uses the YOLO
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