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
    var command = task.command;

    // Step 1: Use a regex to find all {{XXX}} instances in the command
    const paramRegex = /\{\{(\w+)\}\}/g;
    let missingParams = [];
    let match;

    // Step 2: Extract parameter names and check if they are provided in flags
    while ((match = paramRegex.exec(command)) !== null) {
      const paramName = match[1]; // Extract parameter name
      if (!flags[paramName]) {
        missingParams.push(paramName); // Add missing parameter name to the list
      } else {
        // Replace the placeholder with the actual value from flags
        command = command.replace(`{{${paramName}}}`, flags[paramName]);
      }
    }

    // Step 3: Check if there are any missing parameters
    if (missingParams.length > 0) {
      console.log(
        `You're missing the mandatory param(s): ${missingParams.join(", ")}`
      );
      return; // Exit the function early
    }

    // Proceed with running the command if all parameters are provided
    Utils.runCommand(command);
  },
});
