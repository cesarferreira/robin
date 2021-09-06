#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const lodash = require("lodash");
// const fs = require("fs");
const Utils = require("../utils/utils");

function replaceAll(str, find, replace) {
  // log(`${str}, ${find}, ${replace}`);
  return str.replace(new RegExp(find, "g"), replace);
}


// Main code //
const self = (module.exports = {
  find: (name, availableCommands) =>
    lodash.filter(availableCommands, (x) => x.name === name),

  run: (task, flags) => {
    Utils.title(`Runing: ${task.name}...`);
    // log(`${task.command}...`);

    var command = task.command;

    let entries = Object.entries(flags);

    for (const value in entries) {
      let variable = entries[value][0];
      let target = entries[value][1];
      if (target) {
        command = replaceAll(command, `{{${variable}}}`, target);
      }
    }

    Utils.runCommand(command);
  },
});
