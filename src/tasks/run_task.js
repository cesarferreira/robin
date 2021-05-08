#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const lodash = require("lodash");
const fs = require("fs");
const Utils = require("../utils/utils");

// Main code //
const self = (module.exports = {
  find: (name, availableCommands) => lodash.filter(availableCommands, (x) => x.name === name),
  
  run: (command) => {
    log(`gonna run: ${command.name} -> ${command.command}`)
  },
});
