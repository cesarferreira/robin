#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const Utils = require("./utils/utils");
const log = console.log;

const InitTask = require("./tasks/init_task");
const ListTask = require("./tasks/list_task");
const RunTask = require("./tasks/run_task");
const InteractiveTask = require("./tasks/interactive_task");
const hasFlag = require("has-flag");

function interruptIfConfigMissing() {
  if (!Utils.configFileExists()) {
    Utils.titleError(
      `${Utils.CONFIG_FILE_NAME} file is missing, please use the 'init' command`
    );
    process.exit();
  }
}

// Main code //
const self = (module.exports = {
  init: (input, flags) => {
    const command = input.join(" ");
    const params = input.subarray(1, input.length);

    const availableCommands = ListTask.getCommandList();

    if (flags !== undefined && Object.keys(flags).length > 0) {
      // log(`hey flags ${JSON.stringify(flags)}`);

      const list = hasFlag("-l") || hasFlag("--list");
      const interactive = hasFlag("-i") || hasFlag("--interactive");

      if (list) {
        interruptIfConfigMissing();
        ListTask.init();
        return;
      }

      if (interactive) {
        interruptIfConfigMissing();
        InteractiveTask.init(input, availableCommands);
        return;
      }
    }

    switch (command.toLowerCase()) {
      case "init":
        InitTask.init(params, flags);
        break;
      // case "list":
      // case "-l":
      //   interruptIfConfigMissing();
      //   ListTask.init(params);
      //   break;
      // case "interactive":
      // case hasFlag("-i"):
      //   interruptIfConfigMissing();
      //   InteractiveTask.init(params, availableCommands);
      //   break;

      default:
        const result = RunTask.find(command, availableCommands);

        if (result.length != 0) {
          RunTask.run(result[0], flags);
        } else {
          log(`Sorry, cant find "${command}" in your .robin.json`);
        }
    }
  },
});
