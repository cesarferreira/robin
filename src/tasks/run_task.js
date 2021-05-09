#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const lodash = require("lodash");
const fs = require("fs");
const Utils = require("../utils/utils");
const { spawn } = require("child_process");
const execFile = require("child_process").execFile;
const es = require("event-stream");
const child_process = require('child_process');


const { exec } = require('child_process');

function runCommand(task) {
  log("COMANDO: "+task.command);
 
exec(task.command, (err, stdout, stderr) => {
  if (err) {
    //some err occurred
    error(err)
  } else {
    if (!stderr) {

      log(`${stdout}`);
    } else {

      log(`stderr: ${stderr}`);
    }
  }
});
}


function execute(task) {
  const splitArray = task.command.split(" ");
  const command = splitArray[0].trim();
  const params = task.command
    .substring(command.length, task.command.length)
    .trim();

  log(command + " | " + params);

  // const params = [];
  // log(`Running: ${Chalk.green.bold(params.join(' '))}\n`);
  // spawn(command, [], {stdio: 'inherit'});
  spawn(task.command, [], { stdio: "inherit" });
}

// Main code //
const self = (module.exports = {
  find: (name, availableCommands) =>
    lodash.filter(availableCommands, (x) => x.name === name),

  run: (task) => {
    Utils.title(`Runing: ${task.name}...`);

    const splitArray = task.command.split(" ");
    const command = splitArray[0].trim();
    const params = task.command
      .substring(command.length, task.command.length)
      .trim();

    // log(command + " " + params);
    // execute(task);

    runCommand(task);
  },
});
