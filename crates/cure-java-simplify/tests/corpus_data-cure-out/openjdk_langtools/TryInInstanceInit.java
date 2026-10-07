class TryInInstanceInit {
    {
        try {} catch (Exception e) {}
        synchronized (this) {}
    }
}
