#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const fs = require("fs");
// const table = require("table");
const { table, getBorderCharacters } = require( 'table');

const Utils = require("../utils/utils");

// Main code //
const self = (module.exports = {
  // MOVE ME TO UTILS?
  readConfigFile: () => fs.readFileSync(Utils.getConfigPath()),
  getCommandList: () => {
    let commands = [];

    if (Utils.configFileExists()) {
      let config = JSON.parse(self.readConfigFile());

      let entries = Object.entries(config.scripts);

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
    const data = [
      // ["0A", "0B"],
      // ["1A", "1B"],
      // ["2A", "2B"],
    ];
    for (var c of commands) {
      // log(
      //   Chalk.blue("==>") +
      //     Chalk.bold(` ${c.name}  `) +
      //     Chalk.grey(`# ${c.command}`)
      // );

      data.push([Chalk.blue("==>") + Chalk.bold(` ${c.name}`), Chalk.grey(`# ${c.command}`)]);
    }

    const output = table(data, {
      border: getBorderCharacters("void"),
      columnDefault: {
        paddingLeft: 0,
        paddingRight: 4,
      },
      drawHorizontalLine: () => false,
    });

    console.log(output);
  },
});
