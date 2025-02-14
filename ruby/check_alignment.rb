require 'pathname'
require 'json'

here = Pathname __dir__

text = File.read here + '../alignment.json'
xs = JSON.load text

str = xs.map { |x|
  text = x['s']
  text = "|#{text}" if x['st']
  text = "#{text}|" if x['et']
  text
}.join

puts str

File.write here + '../preview.md', str
