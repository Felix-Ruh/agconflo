local given, host = ...
local help = require('help')
local lines, trim = help.lines, help.trim
local base = table.concat({
  help.NO_TOOLS .. "Below is the introduction of the file the goal goes in. It names the themes the file's goals cover, as short lowercase clauses. If the goal below adds a theme the introduction does not name yet, write the introduction again with that theme added as one more clause in the same style, changing nothing else, wrapped at 79 columns. Otherwise answer exactly UNCHANGED. Answer with the paragraph or UNCHANGED and nothing else - no heading, no quotes, no commentary." .. help.GOAL_NOTE,
  "\n\n## goal\n\n" .. given.goal:render(),
  "\n\n## introduction\n\n" .. given.introduction:render(),
  "\n\n## file_goals\n\n" .. given.file_goals:render()
})
return help.ask(host, 'drafting', base, function(out, need, clean)
  clean(out, "the introduction")
  if trim(out) ~= "UNCHANGED" then
    local n, first = 0, lines(given.introduction:render())[1] or ""
    for _, l in ipairs(lines(trim(out))) do
      n = n + 1
      need(#l <= 79, "line " .. n .. " is longer than 79 columns")
      need(not l:match("^%.%. ") and not l:match("^   :"), "line " .. n .. " is not prose")
    end
    need(n <= 10, "it is longer than ten lines")
    local s = out:gsub("``[^`]*``", "")
    need(not s:find(", [A-Z]"), "a clause after a comma starts with a capital letter: every theme is a lowercase clause like the others")
    need(trim(out):sub(1, 30) == first:sub(1, 30), "it does not begin as the introduction does: " .. first:sub(1, 30))
  end
end)
