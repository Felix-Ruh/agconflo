local given, host = ...
local help = require('help')
local lines, trim = help.lines, help.trim
local st = "\n" .. given.state:render()
local k = tonumber(st:match("\ngoal (%d+)\n") or "1")
local p = tonumber(st:match("\npass (%d+)\n") or "1")
local stage = st:match("\nstage (%a+)\n") or "no"
local fb = st:match("feedback:\n(.*)$") or ""
if trim(fb) == "" then fb = "none" end
local ss = {}
for _, l in ipairs(lines(given.statements:render())) do if trim(l) ~= "" then ss[#ss+1] = trim(l) end end
local ws = lines(given.given_stakeholders:render())
return host.text(host.output, "GOAL " .. k .. " OF " .. #ss .. "\nPASS " .. p .. "\nSTAGE " .. stage ..
  "\nSTATEMENT: " .. (ss[k] or "") .. "\nSTAKEHOLDER GIVEN: " .. trim(ws[k] or "unset") .. "\nFEEDBACK:\n" .. fb)
