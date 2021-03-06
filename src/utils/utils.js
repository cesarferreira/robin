#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const fs = require("fs");

Array.prototype.subarray = function (start, end) {
  if (!end) {
    end = -1;
  }
  return this.slice(start, this.length + 1 - end * -1);
};

// Main code //
const self = (module.exports = {
  getConfigPath: () => process.cwd() + "/" + self.CONFIG_FILE_NAME,
  configFileExists: () => fs.existsSync(self.getConfigPath()),
  copyFileHere: (sourceFile) => fs.copyFileSync(sourceFile, `./${self.CONFIG_FILE_NAME}`), 
  copyConfigFileHere: (configName) => self.copyFileHere(`${__dirname}/../../template/${configName}-robin.config.json`), 
  isEmpty: (obj) => Object.keys(obj).length === 0,
  saveToFile: (content, filePath) => fs.writeFileSync(filePath, content, "utf-8"),
  readFile: (filePath) => fs.readFileSync(filePath, "utf-8"),
  title: (text) => log(Chalk.blue("==>") + Chalk.bold(` ${text}`)),
  titleError: (text) => log(Chalk.red("==>") + Chalk.bold(` ${text}`)),
  runCommand: (command) => {
    const exec = require("child_process").exec;
    exec(command).stdout.pipe(process.stdout);
  },

  CONFIG_FILE_NAME: "robin.json",
});
