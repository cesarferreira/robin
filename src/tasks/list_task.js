#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const fs = require("fs");
const Utils = require("../utils/utils");

// Main code //
const self = (module.exports = {
  // MOVE ME TO UTILS?
  getConfigPath: () => process.cwd() + "/" + Utils.CONFIG_FILE_NAME,
  readConfigFile: () => fs.readFileSync(self.getConfigPath()),
  getCommandList: () => {
    let config = JSON.parse(self.readConfigFile());
    var commands = [];

	let entries = Object.entries(config.scripts);

	for (const value in entries) {
		let name = entries[value][0];
		let command = entries[value][1];
		if (name != undefined){
			commands.push({ name: name, command: command });
		}	
	}

	return commands;
  },
  init: () => {
    let commands = self.getCommandList();
    for (var c of commands) {
      log(" -"+c.name);
    }

  },
});
