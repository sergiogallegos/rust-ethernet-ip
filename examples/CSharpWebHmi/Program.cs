using CSharpWebHmi.Services;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddSingleton<PlcDashboardService>();
builder.Services.AddCors(options =>
{
    options.AddPolicy("FrontendDevelopment", policy =>
        policy.WithOrigins("http://127.0.0.1:5173", "http://localhost:5173")
            .AllowAnyHeader()
            .AllowAnyMethod());
});

var app = builder.Build();

app.UseCors("FrontendDevelopment");
app.UseDefaultFiles();
app.UseStaticFiles();

app.MapGet("/api/dashboard", async (PlcDashboardService dashboard, CancellationToken cancellationToken) =>
    Results.Ok(await dashboard.GetSnapshotAsync(cancellationToken)));

app.MapPost("/api/dashboard/pulse", async (PlcDashboardService dashboard, CancellationToken cancellationToken) =>
{
    var result = await dashboard.PulseTestBoolAsync(cancellationToken);
    return result.Success ? Results.Ok(result) : Results.BadRequest(result);
});

app.MapGet("/api/health", () => Results.Ok(new { status = "ok" }));
app.MapFallbackToFile("index.html");

app.Run();
