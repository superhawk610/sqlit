# frozen_string_literal: true

require "ffi"
require_relative "sqlit/version"
require_relative "sqlit/parser"

# Top-level entrypoint.
module Sqlit
  class Error < StandardError; end
  extend FFI::Library

  ffi_lib "ffi/target/debug/libsqlit.#{FFI::Platform::LIBSUFFIX}"
  attach_function :say_hello, [], :void

  # Wave to a person!
  #
  # @param name [String] who to wave to
  # @return [void]
  def self.wave(name)
    puts "Hello, #{name}"
  end

  # Launch the REPL.
  def self.repl
    loop do
      printf "sqlit# "
      query = Sqlit::Parser.parse(gets.strip)
      p query
    rescue Sqlit::ParseError => e
      puts "failed to parse query: #{e.message}"
      puts
    end
  end
end
