require 'pathname'
require 'json'

here = Pathname __dir__
input = here + '../alignment.json'
output = here + '../resources/content.json'

output.parent.mkpath unless output.parent.exist?

data = JSON.load File.read input
result = optimize_content data
File.write output, JSON.pretty_generate(result)


BEGIN{
def optimize_content data
  gs = data.group_by { |x| x['l'] }.values

  parts = gs.map { |xs|
    w = xs.find { |x| x['w'] == 1 }
    puts "skipping odd line, probably fake at the end: #{xs.first && xs.first['l']}..." unless w
    next unless w 

    part = xs.map { |x| x['s'] }.join
    seek = w['st']

    { 'part' => part, 'seek' => seek }
  }
  parts.compact!

  result = []

  parts.each { |x|
    if x['seek']
      result << x
    else
      prev = result.last
      if prev
        prev['part'] << x['part'] # get joined
      else
        result << x
      end
    end
  }
  parts = result

  prev = nil
  parts.each { |x|
    spaces = x['part'][/^\s+/]
    if spaces && prev
      prev['part'] << spaces
      x['part'].sub! spaces, ''
    end

    prev = x
  }

  parts
end
}
