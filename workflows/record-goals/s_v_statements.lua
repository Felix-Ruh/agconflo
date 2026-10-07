local given, host = ...
local help = require('help')
local lines, trim = help.lines, help.trim
local check = help.checks(host)
local n = 0
for _, l in ipairs(lines(given.statements:render())) do if trim(l) ~= "" then n = n + 1 end end
check.need(n >= 1, "the brief quotes no statement beginning 'Agconflo shall'")
return check.report("statements copied from the brief (" .. n .. ")")
