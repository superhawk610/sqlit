# frozen_string_literal: true

require_relative "libsqlit"
require_relative "sqlit/version"
require_relative "sqlit/parser"

# Top-level entrypoint.
module Sqlit
  # Launch the REPL.
  def self.repl
    loop do
      printf "sqlit# "
      query = LibSqlit::Query.parse(gets.strip)
      puts query
    rescue Sqlit::ParseError => e
      puts "failed to parse query: #{e.message}"
      puts
    end
  end
end
