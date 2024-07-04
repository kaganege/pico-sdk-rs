function(to_toml_array STR TOML PAT)
  string(STRIP "${STR}" STR)
  string(REGEX REPLACE "(${PAT})+" "${PAT}" STR "${STR}")
  # Escape double quotes
  string(REPLACE [["]] [[\"]] STR "${STR}")
  string(REPLACE "${PAT}" "\", \"" STR "${STR}")

  set(${TOML} "[\"${STR}\"]" PARENT_SCOPE)
endfunction()
