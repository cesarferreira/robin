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
  readConfigFile: () => require(process.cwd() + "/" + CONFIG_FILE_NAME),
  getCommandList: () => JSON.parse(self.readConfigFile()).scripts,
  init: () => {
    const path = self.getCommandList();
    log(`${path}`);
  },
});
