#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const lodash = require("lodash");
const fs = require("fs");
const Utils = require("../utils/utils");
const { spawn } = require("child_process");
const child_process = require("child_process");

const { exec } = require("child_process");

function replaceAll(str, find, replace) {
  log(`${str}, ${find}, ${replace}`);
  return str.replace(new RegExp(find, "g"), replace);
}

function runCommand(command) {
  exec(command, (err, stdout, stderr) => {
    if (err) {
      //some err occurred
      error(err);
    } else {
      if (!stderr) {
        log(`${stdout}`);
      } else {
        log(`stderr: ${stderr}`);
      }
    }
  });
}

// Main code //
const self = (module.exports = {
  find: (name, availableCommands) =>
    lodash.filter(availableCommands, (x) => x.name === name),

  run: (task, flags) => {
    Utils.title(`Runing: ${task.name}...`);
    log(`${task.command}...`);

    log(flags);

    var command = task.command;

    let entries = Object.entries(flags);

    for (const value in entries) {
      let variable = entries[value][0];
      let target = entries[value][1];
      if (target) {
        command = replaceAll(command, `{{${variable}}}`, target);
      }
    }

    log(`command is: ${command}`);
    // runCommand(command);
  },
});
