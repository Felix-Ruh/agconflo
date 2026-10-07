local given, host = ...
local help = require('help')
local trim = help.trim
local out, k = {}, 0
for _, sent in ipairs(help.body_sentences(given.draft:render())) do
  local low = " " .. sent:lower():gsub("[^%a ]", " ") .. " "
  for _, w in ipairs({ " only ", " each ", " every ", " exactly ", " no other " }) do
    if low:find(w, 1, true) then k = k + 1; out[#out+1] = "W" .. k .. ': "' .. sent .. '" uses "' .. trim(w) .. '"'; break end
  end
end
if #out == 0 then return host.text(host.output, "none") end
return host.text(host.output, table.concat(out, "\n"))
