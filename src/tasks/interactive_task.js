#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const log = console.log;
const fs = require("fs");
const Utils = require("../utils/utils");
const inquirer = require("inquirer");
const MiniSearch = require("minisearch");
const RunTask = require("./run_task");

let availableCommands = [];
let miniSearch;

// Main code //
const self = (module.exports = {
  init: (input, _availableCommands) => {
    availableCommands = _availableCommands || [];

    // Initialize MiniSearch
    miniSearch = new MiniSearch({
      fields: ["name"], // the name field of the commands
      storeFields: ["name", "command"], // store the name field to return it in search results
    });

    // Add commands to MiniSearch index
    miniSearch.addAll(availableCommands.map((cmd, id) => ({ id, ...cmd })));

    inquirer.registerPrompt(
      "autocomplete",
      require("inquirer-autocomplete-prompt")
    );
    inquirer
      .prompt({
        type: "autocomplete",
        name: "command",
        pageSize: 30,
        message: "Which command do you want to run?",
        source: (answersSoFar, input) => {
          return self.fuzzySearch(availableCommands, input);
        },
      })
      .then((choice) => {
        // log(choice);
        const commandObject = availableCommands.find(
          (cmd) => cmd.name === choice.command
        );
        const command = commandObject
          ? commandObject.command
          : "Command not found";

        // log(`${choice.command} - ${command}`);
        RunTask.run(commandObject, {});
      });
  },

  fuzzySearch: (ol, textToFind) => {
    textToFind = textToFind || "";

    return new Promise(function (resolve) {
      let searchOptions = {
        prefix: true,
        fuzzy: 0.2,
      };

      if (textToFind.length == 0) {
        resolve(self.formatCommandList(ol));
      }
      const results = miniSearch.search(textToFind, searchOptions);
      resolve(self.formatCommandList(results));
    });
  },

  formatCommandList: (ol) => {
    if (!Array.isArray(ol) || ol.length === 0) {
      return []; // Return an empty array if ol is not an array or is empty
    }

    const Chalk = require("chalk");
    const longestNameLength = Math.max(
      0,
      ...ol.map((item) => item.name.length)
    ); // Ensure at least 0
    const spacing = " ".repeat(longestNameLength + 4);

    return ol.map((item) => ({
      name: `${item.name}${spacing.slice(item.name.length)}${Chalk.dim(
        `# ${item.command}`
      )}`,
      value: item.name,
    }));
  },
});
