require 'pathname'
require 'json'

here = Pathname __dir__
input = here + '../resources/content.json'
output = here + '../resources/toc.json'

parts = JSON.load File.read input
toc = extract_toc parts
File.write output, JSON.pretty_generate(toc)

BEGIN{
def extract_toc parts
  # temporary wrapper over xs
  toc = {
    level: 0,
    xs: [],
  }
  stack = [toc]

  toc_item = -> { stack.last }

  go_level = -> n {
    while toc_item[][:level] != n
      stack.pop
    end
  }
  add_item = -> x {
    toc_item[][:xs].push x
  }
  add_header = -> level, x {
    go_level.call level - 1
    node = x.merge(level: level, xs: [])
    toc_item[][:xs].push node
    stack.push node
  }

  header_re = /^(#+) (.+)/

  new_paragraph = true

  # NOTE: A.Pope numeration may or may not be useful... have own then?
  # will use indexes probably?
  book_num = 0
  line_num = 0
  para_num = 0
  # fake or special lines is the way to alter generation then through .md

  # counting headers can be done here too?
  # count lines too?
  #
  parts.each_with_index { |p, part_index|
    p['part'].lines.each { |line|
      if line.strip == ''
        new_paragraph = true
        next
      end

      case line
      when header_re
        level = Integer line.match(header_re)[1].chars.count
        text = line.match(header_re)[2].strip

        if level == 2
          book_num += 1
          line_num = 0
          para_num = 0
        end

        add_header.call level, str: text
      else
        line_num += 1
        if new_paragraph
          para_num += 1

          text = shorten line # nice decision
          add_item.call str: text, ref: part_index
          # (no need for num noise since short text is good already)
          #, num: "#{book_num}'#{para_num}"
          # ¶
          #, num: "#{book_num}.#{line_num}"
          new_paragraph = false
        end
      end
    }
  }

  toc[:xs]
end

require 'set'

$connection_words = Set.new(%w[
                            thy o
  a an and are as at be by for from has he in is it its of on or that the to was were will with 
  about above after again against all am any are aren’t because been before being below between 
  both but by can can’t cannot could couldn’t did didn’t do does doesn’t doing don’t down during 
  each few for from further had hadn’t has hasn’t have haven’t having he he’d he’ll he’s her here 
  here’s hers herself him himself his how how’s i i’d i’ll i’m i’ve if in into is isn’t it it’s 
  its itself let’s me more most mustn’t my myself no nor not off on once only or other ought our 
  ours ourselves out over own same shall shan’t she she’d she’ll she’s should shouldn’t so some 
  such than that that’s the their theirs them themselves then there there’s these they they’d 
  they’ll they’re they’ve this those through to too under until up very was wasn’t we we’d we’ll 
  we’re we’ve were weren’t what what’s when when’s where where’s which while who who’s whom why 
  why’s with won’t would wouldn’t you you’d you’ll you’re you’ve your yours yourself yourselves 
  herein hereby heretofore hither henceforth hence
                            ].map{ |x| x.chars.select { |x| x =~ /\w/}.join })

# xxx ascii at some step

def shorten(string)
  got = ''
  string.chars.each { |c|
    got << c

    # at word boundary
    if got.chars.last !~ /\w/
      core = got.chars.select { |x| x =~ /[\w ]/ }.join.downcase
      words = core.scan(/\w+/)
      meaningful_words = words.reject { |x| $connection_words.include? x }

      # if words.count == 5 || meaningful_words.count == 2
      if meaningful_words.count == 2
        got.chop!
        return got.strip + ' …' # not word but sentence got cut
      end
    end
  }
  return got.strip
end
}
