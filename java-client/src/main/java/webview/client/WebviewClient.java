package webview.client;

import com.alibaba.fastjson.JSONObject;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.ByteBuffer;
import java.nio.channels.SocketChannel;
import java.nio.charset.StandardCharsets;
import java.util.Scanner;
import java.util.UUID;
import java.util.function.Consumer;

public class WebviewClient {
    private SocketChannel socketChannel;

    public WebviewClient(int port) throws IOException {
        new Thread(() -> {
            try (SocketChannel socketChannel = SocketChannel.open()) {
                this.socketChannel = socketChannel;
                socketChannel.connect(new InetSocketAddress("localhost", port));

                Receiver receiver = new Receiver(this::onMessageReceived);
                ByteBuffer buffer = ByteBuffer.allocate(256);
                int available;
                while ((available = socketChannel.read(buffer)) != -1) {
                    buffer.flip();
                    receiver.onDataReceived(buffer.array(), available);
                    buffer.clear();
                }
            } catch (IOException e) {
                e.printStackTrace();
            }
        }, "RustWebview message listener.").start();
    }

    /**
     * Send message according to length-data packet protocol.
     *
     * @param message message
     */
    private void send(String message) throws IOException {
        byte[] data = message.getBytes(StandardCharsets.UTF_8);

        // create buffers
        ByteBuffer lengthBuffer = ByteBuffer.allocate(4).putInt(data.length);
        lengthBuffer.flip();
        ByteBuffer messageBuffer = ByteBuffer.wrap(data);

        // write length then message
        while (lengthBuffer.hasRemaining()) {
            socketChannel.write(lengthBuffer);
        }
        while (messageBuffer.hasRemaining()) {
            socketChannel.write(messageBuffer);
        }
    }

    private void onMessageReceived(String message) {
        System.out.println(message);
    }

    private void close() throws IOException {

    }

    public static void main(String[] args) throws IOException {
        WebviewClient webviewClient = new WebviewClient(39174);

        Scanner scanner = new Scanner(System.in);
        System.out.println("Enter messages to send (type 'exit' to quit):");

        while (true) {
            String input = scanner.nextLine();
            if ("exit".equalsIgnoreCase(input)) {
                System.exit(0);
            }
            JSONObject request = new JSONObject();
            request.put("id", UUID.randomUUID().toString());
            request.put("handler", input);
            JSONObject params = new JSONObject();
            request.put("params", params);
            params.put("Browser", 1);

            webviewClient.send(request.toJSONString());
        }
    }
}

class Receiver {
    enum State {
        ForLength, ForData
    }

    private ByteBuffer lengthBuffer;
    private ByteBuffer messageBuffer;
    private State state = State.ForLength;
    private final Consumer<String> messageConsumer;

    public Receiver(Consumer<String> messageConsumer) {
        this.messageConsumer = messageConsumer;
    }

    public void onDataReceived(byte[] data, int available) {
        onDataReceived(data, 0, available);
    }

    private void onDataReceived(byte[] data, int offset, int available) {
        if (offset == available) {
            return;
        }

        switch (state) {
            case ForLength: {
                if (lengthBuffer == null) {
                    lengthBuffer = ByteBuffer.allocate(4);
                }

                int len = Math.min(lengthBuffer.remaining(), data.length - offset);
                lengthBuffer.put(data, offset, len);
                offset += len;

                if (!lengthBuffer.hasRemaining()) {
                    lengthBuffer.flip();
                    int messageLength = lengthBuffer.getInt();
                    messageBuffer = ByteBuffer.allocate(messageLength);
                    lengthBuffer = null;
                    state = State.ForData;
                    onDataReceived(data, offset, available);
                }
                break;
            }
            case ForData: {
                int len = Math.min(messageBuffer.remaining(), data.length - offset);
                messageBuffer.put(data, offset, len);
                offset += len;

                if (!messageBuffer.hasRemaining()) {
                    messageBuffer.flip();
                    String message = StandardCharsets.UTF_8.decode(messageBuffer).toString();
                    messageConsumer.accept(message);
                    messageBuffer = null;
                    state = State.ForLength;
                    onDataReceived(data, offset, available);
                }
                break;
            }
        }
    }
}

