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
      attach_function :free, :sqlit_string_free, [LibSqlit::String], :void
    end
  end

  class Error < StandardError
    extend FFI::Library
    ffi_lib LibSqlit::PATH
    attach_function :last_err, :sqlit_error_last, [], LibSqlit::String

    # Check if an error was generated, and raise it if so.
    def self.check_raise
      err = last_err
      raise Error, err.read_string.force_encoding("UTF-8") unless err.null?
    end
  end

  class DB < FFI::AutoPointer
    def self.open(file)
      db = Binding.open(file)
      LibSqlit::Error.check_raise
      db
    end

    def self.release(ptr)
      Binding.close(ptr)
    end

    def to_s
      # don't cache since DB may be updated
      Binding.debug(self).read_string.force_encoding("UTF-8")
    end

    module Binding
      extend FFI::Library
      ffi_lib LibSqlit::PATH
      attach_function :open, :sqlit_db_open, [:string], DB
      attach_function :debug, :sqlit_db_debug, [DB], LibSqlit::String
      attach_function :close, :sqlit_db_close, [LibSqlit::String], :void
    end
  end

  class Query < FFI::AutoPointer
    def self.parse(input)
      query = Binding.parse(input)
      LibSqlit::Error.check_raise
      query
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
      attach_function :parse, :sqlit_query_parse, [:string], Query
      attach_function :debug, :sqlit_query_debug, [Query], LibSqlit::String
      attach_function :free, :sqlit_query_free, [Query], :void
    end
  end
end
