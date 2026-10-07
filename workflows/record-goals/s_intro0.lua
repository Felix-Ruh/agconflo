local given, host = ...
local help = require('help')
local lines, trim = help.lines, help.trim
local ls, out, started = lines(given.current:render()), {}, false
for i = 2, #ls do
  local l = ls[i]
  if l:match("^%.%. ") then break end
  if not l:match("^=+$") and trim(l) ~= "" and i > 4 then started = true; out[#out+1] = l
  elseif started and trim(l) == "" then break end
end
return host.text(host.output, table.concat(out, "\n"))
