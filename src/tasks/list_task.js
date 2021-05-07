#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const fs = require("fs");
const Utils = require("../utils/utils");

const CONFIG_FILE_NAME = ".robin.config.json";
// Main code //
const self = (module.exports = {
  // MOVE ME TO UTILS?
  getConfigPath: () => process.cwd() + "/" + CONFIG_FILE_NAME,
  readConfigFile: () => fs.readFileSync(self.getConfigPath()),
  getCommandList: () => JSON.parse(self.readConfigFile()),
  init: () => {

	let rawdata = fs.readFileSync(self.getConfigPath());
    // log(rawdata);
    let config = JSON.parse(rawdata);
    // log(config);

	for (const [key, value] of Object.entries(config.scripts)) {
		console.log(key, value);
	  }

    // log(`path: ${self.getConfigPath()}`);
    // log(`configfile: ${self.readConfigFile()}`);
    // log(`commandList: ${self.getCommandList()}`);
  },
});
