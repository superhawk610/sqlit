# frozen_string_literal: true

require "ffi"

module LibSqlit
  PATH = "ffi/target/debug/libsqlit.#{FFI::Platform::LIBSUFFIX}"

  class String < FFI::AutoPointer
    def self.release(ptr)
      Binding.free(ptr)
    end

    module Binding
      extend FFI::Library
      ffi_lib LibSqlit::PATH
      attach_function :free, :sqlit_free_string, [String], :void
    end
  end

  class Query < FFI::AutoPointer
    def self.parse(input)
      Binding.parse(input)
    end

    def self.release(ptr)
      Binding.free(ptr)
    end

    def to_s
      @str ||= Binding.debug(self).read_string.force_encoding("UTF-8")
    end

    module Binding
      extend FFI::Library
      ffi_lib LibSqlit::PATH
      attach_function :parse, :sqlit_parse_query, [:string], Query
      attach_function :debug, :sqlit_debug_query, [Query], LibSqlit::String
      attach_function :free, :sqlit_free_query, [Query], :void
    end
  end
end
