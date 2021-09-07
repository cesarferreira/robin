#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const fs = require("fs");
const Utils = require("../utils/utils");

// Main code //
const self = (module.exports = {
  // MOVE ME TO UTILS?
  readConfigFile: () => fs.readFileSync(Utils.getConfigPath()),
  getCommandList: () => {
    var commands = [];
    if (Utils.configFileExists()) {
      let config = JSON.parse(self.readConfigFile());

      let entries = Object.entries(config.scripts);

      log(entries);

      for (const value in entries) {
        let name = entries[value][0];
        let command = entries[value][1];
        if (name != undefined) {
          commands.push({ name: name, command: command });
        }
      }
    }

    return commands;
  },
  init: () => {
    let commands = self.getCommandList();
    for (var c of commands) {
      log(
        Chalk.blue("==>") +
          Chalk.bold(` ${c.name}  `) +
          Chalk.grey(`# ${c.command}`)
      );
    }
  },
});
