require 'json'
require 'pathname'

# min_duration = 10 # sec.

audio = ARGV.pop or raise "expected: audio-file param"
audio = File.expand_path audio

here = Pathname __dir__

text = File.read here + '../alignment.json'
chunks = JSON.load text

paragraphs = chunks.group_by { |x| x['p'] }.values

# reverse movement, joing-up all 1-line paragraphs (titles)
#
result = []
paragraphs.reverse_each { |xs|
  lines_count = xs.map { |y| y['l'] }.uniq.count
  if lines_count == 1
    # join-up
    place = result.first
    place.unshift *xs
  else
    result.unshift xs
  end
}

paragraphs = result
p paragraphs.map { |xs| xs.map { |y| y['l'] }.uniq.count }

puts '---'
# when no clear boundary - they get merged
cuts = []
prev_end = nil
paragraphs.each { |xs|
  # ignoring white-space/markdown symbols
  finish = xs.reverse.find { |x| x['w'] }
  start = xs.find { |x| x['w'] }

  start_time = start['st']

  if !start_time
    if prev_end && prev_end['et']
      cuts << prev_end['et']
    else
      puts %'ignoring paragraph split without start/prev-end recognized timing: #{start}, #{prev_end}'
    end
  else
    cuts << start_time
  end

  prev_end = finish
}

puts '---'
puts 'Cut times:'
p cuts
puts "Cuts count: %s" % cuts.count

#
# applying cuts to media
#

require 'pathname'

src = audio

resources_dir = here + '../resources'
summary_file = resources_dir + 'cuts.json'
cuts_dir = resources_dir + 'cuts'

cuts_dir.parent.mkpath unless cuts_dir.parent.exist?
# cuts_dir.rmtree if cuts_dir.exist?
cuts_dir.mkpath unless cuts_dir.exist?


files = []

from = 0.0
to = cuts.first
target = cuts_dir + "#{from}.flac"
system "ffmpeg -i #{src.to_s.inspect} -ss #{from} -to #{to} -c:a flac #{target.to_s.inspect}" unless target.exist?
files << [target, from]

cuts.each_cons(2) { |(from, to)|
  target = cuts_dir + "#{from}.flac"
  system "ffmpeg -i #{src.to_s.inspect} -ss #{from} -to #{to} -c:a flac #{target.to_s.inspect}" unless target.exist?
  files << [target, from]
}

from = cuts.last
target = cuts_dir + "#{from}.flac"
system "ffmpeg -i #{src.to_s.inspect} -ss #{from} -c:a flac #{target.to_s.inspect}" unless target.exist?
files << [target, from]

summary = files.map { |(file, from)|
  file = file.relative_path_from(cuts_dir).to_s

  { t: from, f: file }
  # [from, file.relative_path_from(resources_dir).to_s]
}
summary.sort_by! { |x| x[0] }

File.write summary_file, JSON.dump(summary)
