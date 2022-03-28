# frozen_string_literal: true

module Sqlit
  class ParseError < StandardError; end

  class Query
    attr_reader :table

    def initialize(table)
      @table = table
    end
  end

  class SelectQuery < Sqlit::Query
    attr_reader :fields

    def initialize(table:, fields:)
      super(table)
      @fields = fields
    end
  end

  class InsertQuery < Sqlit::Query
    attr_reader :fields, :values

    def initialize(table:, fields:, values:)
      super(table)
      @fields = fields
      @values = values
    end
  end

  class UpdateQuery < Sqlit::Query
    attr_reader :changes, :filter

    def initialize(table:, changes:, filter:)
      super(table)
      @changes = changes
      @filter = filter
    end
  end

  class DeleteQuery < Sqlit::Query
    attr_reader :filter

    def initialize(table:, filter: nil)
      super(table)
      @filter = filter
    end
  end

  class CreateTableQuery < Sqlit::Query
    attr_reader :columns

    def initialize(table:, columns:)
      super(table)
      @columns = columns
    end
  end

  class DropQuery < Sqlit::Query; end

  # SQL query parser.
  class Parser
    def self.parse(input)
      raise ParseError, "no input" if input.empty?

      command, *args = input.downcase.chomp(";").split(" ")
      case command
      when "select"
        k_from, table = args.pop(2)
        raise ParseError, "expected `from` clause" unless k_from == "from"

        Sqlit::SelectQuery.new(table: table, fields: args)
      when "insert"
        k_into, table, k_values = args.shift(3)
        raise ParseError, "expected `into` clause" unless k_into == "into"
        raise ParseError, "expected `values` to be provided" unless k_values == "values"

        args[0].delete_prefix!("(")
        args[-1].delete_suffix!(")")
        values = args.join(" ").split(",").map do |key_value|
          key, value = key_value.split("=").map(&:strip)
          [key, value]
        end

        Sqlit::InsertQuery.new(table: table, values: Hash[values])
      when "update"
        todo!
      when "delete"
        todo!
      when "create"
        todo!
      when "drop"
        todo!
      else
        raise ParseError, "unrecognized command `#{command}`"
      end
    end
  end
end
