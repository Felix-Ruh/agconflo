local given, host = ...
local help = require('help')
local lines, trim = help.lines, help.trim
local check = help.checks(host)
local need = check.need
local d, b = given.fdiff:render(), given.brief:render()
need(given.fubc:render():find("No errors found.", 1, true) ~= nil, "ubc check of the whole change reported errors")
for _, s in ipairs(lines(given.statements:render())) do
  if trim(s) ~= "" then need(d:find("+   :statement: " .. trim(s), 1, true) ~= nil, "not recorded: " .. trim(s)) end
end
local ws = lines(given.given_stakeholders:render())
local k = 0
for _, s in ipairs(lines(given.statements:render())) do
  if trim(s) ~= "" then
    k = k + 1
    local w = trim(ws[k] or "unset")
    if w ~= "unset" then need(d:find("+   :stakeholder: " .. w .. "\n+   :statement: " .. trim(s), 1, true) ~= nil, "not recorded with the stakeholder " .. w .. ": " .. trim(s)) end
  end
end
for f in d:gmatch("\n%+%+%+ b/(%S+)") do need(f:match("^docs/stakeholder/") ~= nil, "changed outside docs/stakeholder: " .. f) end
local named, missing = 0, {}
for id in b:gmatch("[A-Z][A-Z0-9]*_[A-Z0-9_]+") do
  named = named + 1
  if not d:find(id, 1, true) then missing[#missing+1] = id end
end
need(#missing == 0, "needs the brief names but no body names: " .. table.concat(missing, ", "))
return check.report("the whole change: ubc, every statement, only stakeholder files, " .. (named == 0 and "and the brief names no need" or "every need the brief names"))
