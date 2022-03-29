# frozen_string_literal: true

require_relative "sqlit/version"
require_relative "libsqlit"

# Top-level entrypoint.
module Sqlit
  # Launch the REPL.
  def self.repl(db_file)
    begin
      db = LibSqlit::DB.open(db_file)
      puts "connected: #{db}"
    rescue LibSqlit::Error => e
      puts "failed to connect to database: #{e.message}"
      return
    end

    loop do
      printf "sqlit# "
      input = gets

      if input.nil?
        puts
        puts "bye!"
        return
      end

      query = LibSqlit::Query.parse(input.strip)
      puts query
    rescue LibSqlit::Error => e
      puts "failed to parse query: #{e.message}"
      puts
    end
  end
end
