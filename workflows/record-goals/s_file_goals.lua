local given, host = ...
local lines = require('help').lines
local out, title, id = {}, nil, nil
for _, l in ipairs(lines(given.current:render())) do
  local t = l:match("^%.%. stkh_req:: (.*)$")
  if t then title = t end
  local i = l:match("^   :id: (STKH_[A-Z0-9_]+)")
  if i then id = i end
  local who = l:match("^   :stakeholder: (%a+)")
  if who and id and title then out[#out+1] = id .. " - " .. title .. " (stakeholder: " .. who .. ")"; title, id = nil, nil end
end
return host.text(host.output, table.concat(out, "\n"))
