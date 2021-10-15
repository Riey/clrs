namespace System.Runtime.Versioning {
    public class TargetFrameworkAttribute: Attribute {
        public string FrameworkName { get; set; }
        public string FrameworkDisplayName { get; set; }

        public TargetFrameworkAttribute(string frameworkName) {
            this.FrameworkName = frameworkName;
        }
    }
}
