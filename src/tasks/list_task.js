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
  getCommandList: () => {
    let config = JSON.parse(self.readConfigFile());
    var commands = [];

    for (const [key, value] of Object.entries(config.scripts)) {
		let content = Object.entries(value);
      commands.push({ name: content[0][0], command: content[0][1] });
    }
	return commands;
  },
  init: () => {
    let commands = self.getCommandList();
    log(commands);

  },
});
