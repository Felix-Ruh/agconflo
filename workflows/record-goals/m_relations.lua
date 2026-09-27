local given, host = ...

local function lines(s) local t = {} for l in (s .. "\n"):gmatch("(.-)\n") do t[#t+1] = l end return t end
local function trim(s) return (s:gsub("^%s+", ""):gsub("%s+$", "")) end
local fails = {}
local function need(ok, what) if not ok then fails[#fails+1] = what end end
local function clean(s, what)
  need(not (s:find("DSML", 1, true) or s:find("<invoke", 1, true) or s:find("<\239\189\156", 1, true)),
       what .. " is tool-call markup, not the plain text asked for")
  need(trim(s) ~= "", what .. " is empty")
end
local function suggest(id, goals)
  local words, best, bid, btitle = {}, 0, nil, nil
  for w in id:lower():gmatch("[a-z]+") do if #w > 2 and w ~= "stkh" then words[#words+1] = w end end
  for gid, title in (goals or ""):gmatch("\n(STKH_[A-Z0-9_]+)%s+(.-)%s%s+") do
    local score, t = 0, title:lower()
    for _, w in ipairs(words) do if t:find(w, 1, true) then score = score + 1 end end
    if score > best then best, bid, btitle = score, gid, title end
  end
  if bid then return " (did you mean " .. bid .. ', "' .. btitle .. '"?)' end
  return ""
end
local function report(name)
  if #fails == 0 then return host.text(host.output, "PASS " .. name) end
  return host.text(host.output, "FAIL " .. name .. ": " .. table.concat(fails, "; "))
end
local base = table.concat({
  "You have no tools in this step. Do not call a tool and do not write out a tool call in any syntax: answer in plain text only, in exactly the form asked for below.\n\nEach pair below is a sentence from a new requirement's body and the statement of a need it names. For each pair, say whether the sentence describes that need and its relation to the new goal correctly, given only what the need's statement says: one line each, 'P<k>: CORRECT' or 'P<k>: WRONG - <what the statement says instead>'. If the pairs are 'none', answer exactly NONE.",
  "\n\n## pairs\n\n" .. given.pairs:render()
})
local out, why = nil, nil
for attempt = 1, 3 do
  local text = base
  if out then text = text .. "\n\n## your previous answer\n\n" .. out .. "\n\n## why it was refused\n\n" .. why .. "\n\nWrite the whole answer again, correcting every point." end
  out = host.complete('reviewing', host.text(host.output, text)):render()
  fails = {}

  clean(out, "the answer")
  local p = given.pairs:render()
  if trim(p) == "none" then need(trim(out) == "NONE", "there are no pairs, so the answer is NONE")
  else
    for k in p:gmatch("P(%d+):") do
      need(out:find("P" .. k .. ": CORRECT", 1, true) or out:find("P" .. k .. ": WRONG", 1, true), "P" .. k .. " is not answered CORRECT or WRONG")
    end
  end

  if #fails == 0 then break end
  why = table.concat(fails, "\n")
end
return host.text(host.output, out)
