#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const fs = require("fs");
const Utils = require("../utils/utils");

// Main code //
const self = (module.exports = {
  init: (input, flags) => {
    
    if (Utils.configFileExists()) {
      Utils.titleError(`File already exists, please "rm -rf ${Utils.CONFIG_FILE_NAME}"`);
      process.exit();
    }
	
	Utils.copyConfigFileHere('sample'); // the normal one
	// TODO: other envs
	
	Utils.title(`File ${Chalk.green((process.cwd() + '/' + Utils.CONFIG_FILE_NAME))} created`);

  },
});
