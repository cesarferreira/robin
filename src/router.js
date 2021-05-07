#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const Utils = require("./utils/utils");
const log = console.log;

const InitTask = require("./tasks/init_task");
const ListTask = require("./tasks/list_task");

// Main code //
const self = (module.exports = {
  init: (input, flags) => {
    const command = input[0];
    const params = input.subarray(1, input.length);

    switch (command.toLowerCase()) {
      case "init":
		  // TODO if file exists, REFUSE TO do it
        InitTask.init(params);
        break;
      case "list":
        ListTask.init(params);
        break;

      default:
        log(`Sorry, cant find ${command}`);
    }
  },
});
