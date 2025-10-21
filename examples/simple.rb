require 'socket'
s = TCPSocket.new(ENV['HOST'] || 'localhost', 6606)
s.puts "LOGIN #{`whoami`.chomp}@#{`hostname`.chomp}"
s.puts "TITLE: Ruby notificationd client test"
s.puts "SEND"

while (recv = s.gets) do
  abort "Error #{recv}" if recv.start_with? "-"
  exit if recv.start_with? "+SEND"
end

abort "Connection closed unexpectedly"
