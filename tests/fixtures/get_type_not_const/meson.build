sources = [
  'basic.h',
]

foreach src : sources
  static_library('get_type_not_const_' + src.underscorify(),
    src,
    dependencies: fixture_deps,
    c_args: fixture_c_args,
  )
endforeach
