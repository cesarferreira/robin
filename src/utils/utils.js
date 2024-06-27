#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const fs = require("fs");
const { spawn } = require("child_process");

Array.prototype.subarray = function (start, end) {
  if (!end) {
    end = -1;
  }
  return this.slice(start, this.length + 1 - end * -1);
};

function executeCommand(commandString) {
  // Split the command string into an array
  const [command, ...args] = commandString.split(" ");

  const process = spawn(command, args, {
    stdio: "inherit", // This option will print the output of the command to the console
    shell: true, // This option is required for commands to be executed in a shell
  });

  process.on("close", (code) => {
    // console.log(`Process exited with code ${code}`);
  });

  process.on("error", (err) => {
    console.error(`Failed to start subprocess: ${err.message}`);
  });
}

// Main code //
const self = (module.exports = {
  getConfigPath: () => process.cwd() + "/" + self.CONFIG_FILE_NAME,
  configFileExists: () => fs.existsSync(self.getConfigPath()),
  copyFileHere: (sourceFile) =>
    fs.copyFileSync(sourceFile, `./${self.CONFIG_FILE_NAME}`),
  copyConfigFileHere: (configName) =>
    self.copyFileHere(
      `${__dirname}/../../template/${configName}-robin.config.json`
    ),
  isEmpty: (obj) => Object.keys(obj).length === 0,
  saveToFile: (content, filePath) =>
    fs.writeFileSync(filePath, content, "utf-8"),
  readFile: (filePath) => fs.readFileSync(filePath, "utf-8"),
  title: (text) => log(Chalk.blue("==>") + Chalk.bold(` ${text}`)),
  titleError: (text) => log(Chalk.red("==>") + Chalk.bold(` ${text}`)),
  runCommand: (command) => executeCommand(command),

  CONFIG_FILE_NAME: ".robin.json",
});
