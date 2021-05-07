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
  readConfigFile: () => require(self.getConfigPath()),
  getCommandList: () => JSON.parse(self.readConfigFile()),
  init: () => {
	  log(`path: ${self.getConfigPath()}`);
	  log(`configfile: ${self.readConfigFile()}`);
	  log(`commandList: ${self.getCommandList()}`);
  },
});
