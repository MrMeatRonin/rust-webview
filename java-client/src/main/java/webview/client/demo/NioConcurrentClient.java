package webview.client.demo;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.ByteBuffer;
import java.nio.channels.SelectionKey;
import java.nio.channels.Selector;
import java.nio.channels.SocketChannel;
import java.nio.charset.StandardCharsets;
import java.util.Iterator;
import java.util.Scanner;
import java.util.Set;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;

public class NioConcurrentClient {
    private static final String HOST = "127.0.0.1";
    private static final int PORT = 39174;
    private static final BlockingQueue<String> messageQueue = new ArrayBlockingQueue<>(100);
    private static Selector selector;

    public static void main(String[] args) {
        try {
            // Open selector and channel
            selector = Selector.open();
            SocketChannel channel = SocketChannel.open();
            channel.configureBlocking(false);

            // Register for connect operation
            channel.connect(new InetSocketAddress(HOST, PORT));
            channel.register(selector, SelectionKey.OP_CONNECT);

            // Start user input thread
            new Thread(NioConcurrentClient::handleUserInput).start();

            while (true) {
                selector.select();
                Set<SelectionKey> selectedKeys = selector.selectedKeys();
                Iterator<SelectionKey> iter = selectedKeys.iterator();

                while (iter.hasNext()) {
                    SelectionKey key = iter.next();
                    iter.remove();

                    if (key.isConnectable()) {
                        handleConnect(key);
                    } else if (key.isWritable()) {
                        handleWrite(key);
                    } else if (key.isReadable()) {
                        handleRead(key);
                    }
                }
            }
        } catch (IOException e) {
            System.err.println("Client error: " + e.getMessage());
        } finally {
            if (selector != null) {
                try {
                    selector.close();
                } catch (IOException e) {
                    System.err.println("Error closing selector: " + e.getMessage());
                }
            }
        }
    }

    private static void handleConnect(SelectionKey key) throws IOException {
        SocketChannel channel = (SocketChannel) key.channel();
        if (channel.isConnectionPending()) {
            channel.finishConnect();
        }
        System.out.println("Connected to server at " + HOST + ":" + PORT);

        // Now we can write and read
        channel.register(key.selector(), SelectionKey.OP_READ | SelectionKey.OP_WRITE);
    }

    private static void handleWrite(SelectionKey key) throws IOException {
        SocketChannel channel = (SocketChannel) key.channel();
        String message = messageQueue.poll();

        if (message != null) {
            byte[] messageBytes = message.getBytes(StandardCharsets.UTF_8);
            ByteBuffer lengthBuffer = ByteBuffer.allocate(4).putInt(messageBytes.length);
            lengthBuffer.flip();

            ByteBuffer messageBuffer = ByteBuffer.wrap(messageBytes);

            // Write length then message
            while (lengthBuffer.hasRemaining()) {
                channel.write(lengthBuffer);
            }
            while (messageBuffer.hasRemaining()) {
                channel.write(messageBuffer);
            }

            System.out.println("Sent: " + message);
        }
    }

    private static void handleRead(SelectionKey key) throws IOException {
        SocketChannel channel = (SocketChannel) key.channel();

        // Read message length (4 bytes)
        ByteBuffer lengthBuffer = ByteBuffer.allocate(4);
        readFully(channel, lengthBuffer);
        lengthBuffer.flip();
        int length = lengthBuffer.getInt();

        // Read message
        ByteBuffer messageBuffer = ByteBuffer.allocate(length);
        readFully(channel, messageBuffer);
        messageBuffer.flip();

        String response = StandardCharsets.UTF_8.decode(messageBuffer).toString();
        System.out.println("Received: " + response);
    }

    private static void readFully(SocketChannel channel, ByteBuffer buffer) throws IOException {
        while (buffer.hasRemaining()) {
            if (channel.read(buffer) == -1) {
                throw new IOException("Server closed connection");
            }
        }
    }

    private static void handleUserInput() {
        Scanner scanner = new Scanner(System.in);
        System.out.println("Enter messages to send (type 'exit' to quit):");

        while (true) {
            String input = scanner.nextLine();
            if ("exit".equalsIgnoreCase(input)) {
                System.exit(0);
            }
            try {
                messageQueue.put(input);
                // Wake up selector to process the new message
                if (selector != null) {
                    selector.wakeup();
                }
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                break;
            }
        }
    }
}