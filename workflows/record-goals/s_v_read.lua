local given, host = ...
local check = require('help').checks(host)
local need = check.need
need(#given.rules:render() > 400 and given.rules:render():lower():find("requirement", 1, true) ~= nil, "no section about requirements came back from AGENTS.md or README.md")
need(given.example:render():find(":statement:", 1, true) ~= nil, "the example goal has no statement")
need(given.goals:render():find("STKH_", 1, true) ~= nil, "no stakeholder goal came back from the graph")
need(given.stakeholders:render():find("enum", 1, true) ~= nil, "the stakeholder field's definition was not found")
return check.report("the repository was read")
